run:
	@make -C os run

clean:
	@make -C os clean

fmt:
	@cd os && cargo fmt