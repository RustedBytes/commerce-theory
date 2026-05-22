.PHONY: coverage coverage-html

COVERAGE_MIN_LINES ?= 99

coverage:
	cargo llvm-cov --all-features --summary-only --fail-under-lines $(COVERAGE_MIN_LINES)

coverage-html:
	cargo llvm-cov --all-features --html --fail-under-lines $(COVERAGE_MIN_LINES)
