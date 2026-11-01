deploy:
	out=$$(nix build ..#www --no-link --print-out-paths) && \
	mc cp --recursive $$out/ garage/comrak.ee/ && \
	mc mirror --remove $$out/ garage/comrak.ee/
