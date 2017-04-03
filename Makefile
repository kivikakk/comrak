docker:
	docker build -t comrak $(CURDIR)/script
	docker run --privileged -t -i -v $(CURDIR):/src/comrak -v $(HOME)/.cargo/registry:/root/.cargo/registry -w /src/comrak comrak /bin/bash
