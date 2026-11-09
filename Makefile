.PHONY: all fmt clippy test visits doc a11y file-length links zero-style package package-wasm package-wasm-publish check check-ecosystem deny audit machete miri fuzz bench bench-instructions wasm examples clean php

all: check

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --locked

visits:
	cargo test -p rich-editor-auy --features visits --locked --test visits

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

a11y:
	cargo test -p rich-editor-auy --locked --test a11y

file-length:
	./scripts/check_file_length.sh

links:
	python3 scripts/check_links.py

zero-style:
	./scripts/check_zero_style.sh

package:
	cargo package -p rich-editor-auy --locked --allow-dirty
	cargo package -p rich-editor-auy-macros --locked --allow-dirty

package-wasm:
	# The crates.io index cannot resolve the unpublished core, so the offline gate lists contents.
	cargo package --list -p rich-editor-auy-wasm --locked --allow-dirty > /dev/null

package-wasm-publish:
	cargo package -p rich-editor-auy-wasm --locked --allow-dirty --no-verify

check: fmt clippy test visits doc a11y file-length links zero-style package package-wasm

check-ecosystem:
	@if [ -f packages/ts/deno.json ]; then cd packages/ts && deno task check && deno task publish:dry; fi
	@if [ -f packages/php/composer.json ]; then \
		if command -v composer >/dev/null 2>&1; then \
			cd packages/php && composer validate --strict && composer audit; \
		elif command -v docker >/dev/null 2>&1; then \
			$(MAKE) php; \
		else \
			echo "composer and docker unavailable; PHP gate skipped locally"; \
		fi; \
	fi

deny:
	@if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; else echo "cargo-deny not installed; skipped locally"; fi

audit:
	@if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "cargo-audit not installed; skipped locally"; fi

machete:
	@if command -v cargo-machete >/dev/null 2>&1; then cargo machete; else echo "cargo-machete not installed; skipped locally"; fi

miri:
	@if command -v cargo +nightly miri >/dev/null 2>&1; then cargo +nightly miri test -p rich-editor-auy --test miri_smoke; else echo "miri not installed; skipped locally"; fi

fuzz:
	@if [ -d fuzz ]; then cd fuzz && cargo +nightly fuzz run --fuzz-dir . json_parse -- -max_total_time=30 && cargo +nightly fuzz run --fuzz-dir . a11y_tree -- -max_total_time=30 && cargo +nightly fuzz run --fuzz-dir . apply_step -- -max_total_time=30 && cargo +nightly fuzz run --fuzz-dir . round_trip -- -max_total_time=30 && cargo +nightly fuzz run --fuzz-dir . fragment -- -max_total_time=30; fi

php:
	docker build -t rich-editor-auy-php packages/php
	docker run --rm rich-editor-auy-php

bench:
	cd bench && cargo bench --bench hot --bench wire

bench-instructions:
	./scripts/bench_instructions.sh

wasm:
	./scripts/build_wasm.sh

examples:
	deno check examples/ts/main.ts
	deno run --allow-read examples/ts/main.ts

clean:
	cargo clean
