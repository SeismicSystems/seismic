.PHONY: build test fmt fmt-check sync-artifacts check-artifacts ci

# We skip the DepositContract because it stack-too-deeps without --via-ir, and our ssolc compiler
# still doesn't support --via-ir: https://github.com/SeismicSystems/seismic-solidity/issues/195
# If need be, the contract can be compiled using standard forge/solc toolchain:
#   forge build src/seismic-std-lib/DepositContract.sol --via-ir

build:
	sforge build --skip DepositContract

test:
	sforge test --skip DepositContract

fmt:
	sforge fmt

fmt-check:
	sforge fmt --check

sync-artifacts:
	bash script/sync-artifacts.sh

check-artifacts: sync-artifacts
	@git diff --exit-code artifacts/ || \
		(echo "Artifacts are out of date. Run 'make sync-artifacts' and commit." && exit 1)

ci: fmt-check check-artifacts test
