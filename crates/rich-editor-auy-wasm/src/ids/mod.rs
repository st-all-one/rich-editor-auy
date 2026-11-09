//! Anchor policies applied to structured imports.

mod remap;

use std::sync::Arc;

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::{
    AtomData, Attr, AttrId, AttrValue, Attrs, Block, BlockContent, Doc, Fragment, NodeId,
};
use rich_editor_auy::schema::Schema;

use crate::problem;

use remap::{Remap, remap_attrs, rewrite_text};

/// One anchor rewrite produced by the `regenerate` policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdPair {
    /// Anchor present in the payload.
    pub(crate) from: String,
    /// Deterministic anchor assigned to the node.
    pub(crate) to: String,
}

/// Policy applied to anchors carried by an import payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IdPolicy {
    /// Keep anchors exactly as received.
    Preserve,
    /// Replace every anchor with a deterministic identifier.
    Regenerate(String),
    /// Reject payloads that carry anchors.
    Reject,
}

impl IdPolicy {
    /// Parses the requested policy.
    ///
    /// # Errors
    ///
    /// Returns a policy problem for unknown names, a missing seed, or a seed that is too long.
    pub(crate) fn parse(name: Option<&str>, seed: Option<&str>) -> Result<Self, ProblemDetails> {
        match name.unwrap_or("preserve") {
            "preserve" => Ok(Self::Preserve),
            "reject" => Ok(Self::Reject),
            "regenerate" => {
                let Some(seed) = seed.filter(|value| !value.is_empty()) else {
                    return Err(problem::id_policy_problem(
                        "idPolicy \"regenerate\" requires a non-empty idSeed",
                    ));
                };
                if seed.len() > 256 {
                    return Err(problem::id_policy_problem(
                        "idSeed is longer than 256 bytes",
                    ));
                }
                Ok(Self::Regenerate(String::from(seed)))
            }
            other => Err(problem::id_policy_problem(&format!(
                "idPolicy \"{other}\" is not preserve, regenerate, or reject"
            ))),
        }
    }
}

/// Applies the policy to every top-level block of a document.
///
/// `regenerate` rewrites block anchors and every `Ref` value in node, atom, and mark attributes
/// that targets one of them, so internal references keep pointing at the imported content.
///
/// # Errors
///
/// Returns an anchor rejection when the payload carries anchors and the policy rejects them.
pub(crate) fn rewrite_doc(
    doc: &Doc,
    policy: &IdPolicy,
    schema: &Schema,
) -> Result<(Doc, Vec<IdPair>), ProblemDetails> {
    if matches!(policy, IdPolicy::Preserve) {
        return Ok((doc.clone(), Vec::new()));
    }
    let mut map = Vec::new();
    let mut remap = Remap::new();
    let mut path = Vec::new();
    let current = doc.blocks().to_vec();
    collect_blocks(&current, policy, &mut map, &mut remap, &mut path)?;
    let mut path = Vec::new();
    let blocks = rewrite_blocks(&current, &remap, schema, &mut path)?;
    let attrs = match remap_attrs(doc.attrs(), &remap)? {
        Some(attrs) => attrs,
        None => doc.attrs().clone(),
    };
    let rewritten =
        Doc::from_blocks(attrs, blocks).map_err(|error| problem::from_limit_error(&error))?;
    Ok((rewritten, map))
}

/// Applies the policy to every block of a fragment.
///
/// # Errors
///
/// Returns an anchor rejection when the payload carries anchors and the policy rejects them.
pub(crate) fn rewrite_fragment(
    fragment: &Fragment,
    policy: &IdPolicy,
    schema: &Schema,
) -> Result<(Fragment, Vec<IdPair>), ProblemDetails> {
    if matches!(policy, IdPolicy::Preserve) {
        return Ok((fragment.clone(), Vec::new()));
    }
    let mut map = Vec::new();
    let mut remap = Remap::new();
    let mut path = Vec::new();
    collect_blocks(fragment.blocks(), policy, &mut map, &mut remap, &mut path)?;
    let mut path = Vec::new();
    let blocks = rewrite_blocks(fragment.blocks(), &remap, schema, &mut path)?;
    Ok((Fragment::new(blocks), map))
}

/// Collects every anchor rewrite before the tree is rebuilt, so forward references resolve.
fn collect_blocks(
    blocks: &[Arc<Block>],
    policy: &IdPolicy,
    map: &mut Vec<IdPair>,
    remap: &mut Remap,
    path: &mut Vec<u32>,
) -> Result<(), ProblemDetails> {
    for (index, block) in blocks.iter().enumerate() {
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        if let Some(old) = block.id() {
            match policy {
                IdPolicy::Preserve => {}
                IdPolicy::Reject => return Err(problem::id_rejected_problem(&instance(path))),
                IdPolicy::Regenerate(seed) => {
                    let Some(new) = NodeId::derived(seed, path) else {
                        return Err(problem::id_policy_problem(
                            "derived id is not representable",
                        ));
                    };
                    map.push(IdPair {
                        from: String::from(old.as_str()),
                        to: String::from(new.as_str()),
                    });
                    remap.insert(Box::from(old.as_str()), Box::from(new.as_str()));
                }
            }
        }
        if let BlockContent::Blocks(children) = block.content() {
            collect_blocks(children, policy, map, remap, path)?;
        }
        path.pop();
    }
    Ok(())
}

fn rewrite_blocks(
    blocks: &[Arc<Block>],
    remap: &Remap,
    schema: &Schema,
    path: &mut Vec<u32>,
) -> Result<Vec<Arc<Block>>, ProblemDetails> {
    let mut out = Vec::with_capacity(blocks.len());
    for (index, block) in blocks.iter().enumerate() {
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        let mut attrs = match block.id().and_then(|old| remap.get(old.as_str())) {
            Some(new) => block
                .attrs()
                .with(Attr {
                    id: AttrId::NODE_ID,
                    value: AttrValue::Str(Box::from(&**new)),
                })
                .ok_or_else(|| problem::id_policy_problem("duplicate node id attribute"))?,
            None => block.attrs().clone(),
        };
        if let Some(remapped) = remap_attrs(&attrs, remap)? {
            attrs = remapped;
        }
        let content = match block.content() {
            BlockContent::Text(text) => BlockContent::Text(rewrite_text(text, remap, schema)?),
            BlockContent::Atom(atom) => BlockContent::Atom(AtomData {
                attrs: match remap_attrs(&atom.attrs, remap)? {
                    Some(attrs) => attrs,
                    None => atom.attrs.clone(),
                },
            }),
            BlockContent::Blocks(children) => {
                let nested = rewrite_blocks(children, remap, schema, path)?;
                BlockContent::Blocks(nested.into_boxed_slice())
            }
        };
        path.pop();
        out.push(Arc::new(rebuilt(block, attrs, content)));
    }
    Ok(out)
}

/// Rebuilds a block with new attributes while preserving its payload.
fn rebuilt(block: &Block, attrs: Attrs, content: BlockContent) -> Block {
    match content {
        BlockContent::Text(text) => Block::text(block.kind(), attrs, text),
        BlockContent::Atom(atom) => Block::atom(block.kind(), attrs, atom),
        BlockContent::Blocks(children) => {
            Block::container(block.kind(), attrs, children.into_vec())
        }
    }
}

/// Formats a block path for a problem instance.
fn instance(path: &[u32]) -> String {
    let mut out = String::from("blocks");
    for index in path {
        out.push('[');
        out.push_str(&index.to_string());
        out.push(']');
    }
    out
}
