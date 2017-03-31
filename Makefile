docker:
	docker build -t comrak $(CURDIR)/script
	docker run --privileged -t -i -v $(CURDIR):/src/comrak -w /src/comrak comrak /bin/bash 
