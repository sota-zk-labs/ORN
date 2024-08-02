.PHONY: cargo dev-tool prune const-fmt
dev-tool:
	cargo install machete
prune:
	cargo machete
const-fmt:
	taplo fmt -o reorder_keys=true src/const_values.toml