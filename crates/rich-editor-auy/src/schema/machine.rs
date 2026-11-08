//! Deterministic simulation of a Thompson NFA for content expressions.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::error::SchemaError;
use crate::model::NodeKind;

use super::expr::Expr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Transition {
    kind: Option<NodeKind>,
    target: usize,
}

#[derive(Clone, Debug, Default)]
struct State {
    transitions: Vec<Transition>,
}

/// Simulated NFA for one content expression.
#[derive(Clone, Debug)]
pub(super) struct ContentMachine {
    states: Vec<State>,
    start: usize,
    accept: usize,
}

/// Compiles a parsed expression by resolving names and groups.
pub(super) fn compile(
    expression: &Expr,
    resolve: &impl Fn(&str) -> Option<Vec<NodeKind>>,
) -> Result<ContentMachine, SchemaError> {
    let mut builder = Builder {
        states: Vec::new(),
        resolve,
    };
    let (start, accept) = builder.fragment(expression)?;
    Ok(ContentMachine {
        states: builder.states,
        start,
        accept,
    })
}

impl ContentMachine {
    /// Returns whether a sequence of child kinds is accepted.
    pub(crate) fn allows(&self, kinds: &[NodeKind]) -> bool {
        let mut current = self.epsilon_closure(BTreeSet::from([self.start]));
        for kind in kinds {
            let mut next = BTreeSet::new();
            for state in &current {
                let Some(node) = self.states.get(*state) else {
                    return false;
                };
                for transition in &node.transitions {
                    if transition.kind == Some(*kind) {
                        next.insert(transition.target);
                    }
                }
            }
            if next.is_empty() {
                return false;
            }
            current = self.epsilon_closure(next);
        }
        current.contains(&self.accept)
    }

    /// Returns whether the expression accepts no children.
    pub(crate) fn can_be_empty(&self) -> bool {
        self.epsilon_closure(BTreeSet::from([self.start]))
            .contains(&self.accept)
    }

    /// Returns the first accepted child kind in schema order.
    pub(crate) fn first_kind(&self) -> Option<NodeKind> {
        let mut first = None;
        for state in self.epsilon_closure(BTreeSet::from([self.start])) {
            let Some(node) = self.states.get(state) else {
                continue;
            };
            for transition in &node.transitions {
                if let Some(kind) = transition.kind {
                    first = Some(first.map_or(kind, |current: NodeKind| current.min(kind)));
                }
            }
        }
        first
    }

    fn epsilon_closure(&self, seed: BTreeSet<usize>) -> BTreeSet<usize> {
        let mut closure = seed;
        let mut pending = closure.iter().copied().collect::<Vec<_>>();
        while let Some(state) = pending.pop() {
            let Some(node) = self.states.get(state) else {
                continue;
            };
            for transition in &node.transitions {
                if transition.kind.is_none() && closure.insert(transition.target) {
                    pending.push(transition.target);
                }
            }
        }
        closure
    }
}

struct Builder<'a, F> {
    states: Vec<State>,
    resolve: &'a F,
}

impl<F> Builder<'_, F>
where
    F: Fn(&str) -> Option<Vec<NodeKind>>,
{
    fn fragment(&mut self, expression: &Expr) -> Result<(usize, usize), SchemaError> {
        match expression {
            Expr::Empty => Ok(self.epsilon_fragment()),
            Expr::Name(name) => self.name_fragment(name),
            Expr::Seq(parts) => self.sequence(parts),
            Expr::Alt(parts) => self.alternatives(parts),
            Expr::Repeat { expr, min, max } => self.repeat(expr, *min, *max),
        }
    }

    fn sequence(&mut self, parts: &[Expr]) -> Result<(usize, usize), SchemaError> {
        let Some(first) = parts.first() else {
            return Ok(self.epsilon_fragment());
        };
        let (start, mut end) = self.fragment(first)?;
        for part in parts.iter().skip(1) {
            let (next_start, next_end) = self.fragment(part)?;
            self.epsilon(end, next_start);
            end = next_end;
        }
        Ok((start, end))
    }

    fn alternatives(&mut self, parts: &[Expr]) -> Result<(usize, usize), SchemaError> {
        let start = self.add_state();
        let end = self.add_state();
        for part in parts {
            let (part_start, part_end) = self.fragment(part)?;
            self.epsilon(start, part_start);
            self.epsilon(part_end, end);
        }
        Ok((start, end))
    }

    fn repeat(
        &mut self,
        expression: &Expr,
        min: u32,
        max: Option<u32>,
    ) -> Result<(usize, usize), SchemaError> {
        let mut chain: Option<(usize, usize)> = None;
        for _ in 0..min {
            let part = self.fragment(expression)?;
            chain = Some(match chain {
                None => part,
                Some((start, previous_end)) => {
                    self.epsilon(previous_end, part.0);
                    (start, part.1)
                }
            });
        }
        if let Some(upper) = max {
            let optional = upper.saturating_sub(min);
            for _ in 0..optional {
                let part = self.optional(expression)?;
                chain = Some(match chain {
                    None => part,
                    Some((start, previous_end)) => {
                        self.epsilon(previous_end, part.0);
                        (start, part.1)
                    }
                });
            }
            chain.map_or_else(|| Ok(self.epsilon_fragment()), Ok)
        } else {
            let star = self.star(expression)?;
            chain.map_or(Ok(star), |(start, previous_end)| {
                self.epsilon(previous_end, star.0);
                Ok((start, star.1))
            })
        }
    }

    fn star(&mut self, expression: &Expr) -> Result<(usize, usize), SchemaError> {
        let start = self.add_state();
        let end = self.add_state();
        let (part_start, part_end) = self.fragment(expression)?;
        self.epsilon(start, end);
        self.epsilon(start, part_start);
        self.epsilon(part_end, part_start);
        self.epsilon(part_end, end);
        Ok((start, end))
    }

    fn optional(&mut self, expression: &Expr) -> Result<(usize, usize), SchemaError> {
        let start = self.add_state();
        let end = self.add_state();
        let (part_start, part_end) = self.fragment(expression)?;
        self.epsilon(start, end);
        self.epsilon(start, part_start);
        self.epsilon(part_end, end);
        Ok((start, end))
    }

    fn name_fragment(&mut self, name: &str) -> Result<(usize, usize), SchemaError> {
        let kinds = (self.resolve)(name).ok_or_else(|| SchemaError::UnknownName {
            name: alloc::boxed::Box::from(name),
        })?;
        if kinds.is_empty() {
            return Err(SchemaError::UnknownName {
                name: alloc::boxed::Box::from(name),
            });
        }
        let start = self.add_state();
        let end = self.add_state();
        for kind in kinds {
            self.transition(start, Some(kind), end);
        }
        self.sort_state(start);
        Ok((start, end))
    }

    fn epsilon_fragment(&mut self) -> (usize, usize) {
        let start = self.add_state();
        let end = self.add_state();
        self.epsilon(start, end);
        (start, end)
    }

    fn add_state(&mut self) -> usize {
        let index = self.states.len();
        self.states.push(State::default());
        index
    }

    fn transition(&mut self, from: usize, kind: Option<NodeKind>, to: usize) {
        if let Some(state) = self.states.get_mut(from) {
            state.transitions.push(Transition { kind, target: to });
        }
    }

    fn epsilon(&mut self, from: usize, to: usize) {
        self.transition(from, None, to);
    }

    fn sort_state(&mut self, state: usize) {
        if let Some(node) = self.states.get_mut(state) {
            node.transitions.sort();
        }
    }
}
