deploy:
	mc mirror --overwrite --remove $$(nix build ..#www --no-link --print-out-paths)/ kala/comrak.ee/
