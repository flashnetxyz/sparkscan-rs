BUILD_PATH = "target"

.PHONY: coverage-codecov
coverage-codecov:
	cargo llvm-cov nextest
	cargo llvm-cov --workspace --codecov --output-path ./codecov.json
	cargo llvm-cov --workspace --cobertura --output-path ./cobertura.xml
