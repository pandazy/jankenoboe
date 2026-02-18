.PHONY: e2e e2e-build e2e-run clean-e2e

E2E_IMAGE := jankenoboe-e2e

# Build and run e2e tests in Docker
e2e: e2e-build e2e-run

# Build the e2e Docker image
e2e-build:
	docker build -f e2e/Dockerfile -t $(E2E_IMAGE) .

# Run the e2e tests
e2e-run:
	docker run --rm $(E2E_IMAGE)

# Remove the e2e Docker image
clean-e2e:
	docker rmi $(E2E_IMAGE) 2>/dev/null || true