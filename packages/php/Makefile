SHELL := /bin/bash
.DEFAULT_GOAL := help

DOCKER ?= docker

VERSIONS := 8.2 8.3 8.4 8.5
ALPINE_VERSION ?= 8.2

DOCKER_RUN = $(DOCKER) run --rm \
	--security-opt label=disable \
	-u "$$(id -u):$$(id -g)" \
	-e HOME=/tmp \
	-e COMPOSER_ROOT_VERSION=0.1.0 \
	-v "$(CURDIR):/app" \
	-w /app

.PHONY: help build build-pcov build-alpine install update check check-matrix check-alpine \
	coverage smoke fmt shell exec clean

help: ## Show available targets
	@grep -hE '^[a-zA-Z0-9_-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk -F '## ' '{ split($$1, target, ":"); printf "  %-16s %s\n", target[1], $$2 }'

build-%: ## Build the glibc image for one PHP version (make build-8.2)
	$(DOCKER) build -f docker/bookworm.Dockerfile \
		--build-arg PHP_TAG=$*-cli-bookworm \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "$*" { print $$2 }' docker/images.tsv) \
		-t rich-editor-auy-php:$*-bookworm .

build-pcov: ## Build the coverage image (PHP 8.2 + PCOV)
	$(DOCKER) build -f docker/bookworm.Dockerfile \
		--build-arg PHP_TAG=8.2-cli-bookworm \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "8.2" { print $$2 }' docker/images.tsv) \
		--build-arg WITH_PCOV=1 \
		-t rich-editor-auy-php:8.2-pcov .

build-alpine: ## Build the musl image
	$(DOCKER) build -f docker/alpine.Dockerfile \
		--build-arg PHP_TAG=$(ALPINE_VERSION)-cli-alpine \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "$(ALPINE_VERSION)" { print $$3 }' docker/images.tsv) \
		-t rich-editor-auy-php:$(ALPINE_VERSION)-alpine .

build: $(addprefix build-,$(VERSIONS)) build-pcov build-alpine ## Build every gate image

install: build-8.2 ## Install locked Composer dependencies
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer install --no-interaction --prefer-dist --no-progress

update: build-8.2 ## Update Composer dependencies (writes composer.lock)
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer update --no-interaction --prefer-dist --no-progress

check: install ## Full gate on PHP 8.2 (validate, lint, static, test)
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer check

check-matrix: build ## Full gate on 8.2/8.3/8.4/8.5 glibc, coverage and musl
	@for version in $(VERSIONS); do \
		$(DOCKER_RUN) rich-editor-auy-php:$$version-bookworm \
			sh -c 'composer install --no-interaction --prefer-dist --no-progress && composer check' || exit 1; \
	done
	$(MAKE) coverage
	$(MAKE) check-alpine

check-alpine: build-alpine install ## Full gate on musl
	$(DOCKER_RUN) rich-editor-auy-php:$(ALPINE_VERSION)-alpine \
		sh -c 'composer install --no-interaction --prefer-dist --no-progress && composer check'

coverage: build-pcov install ## Run tests with PCOV and enforce 100% statement coverage
	$(DOCKER_RUN) rich-editor-auy-php:8.2-pcov composer coverage

smoke: install ## Install the package in a temporary external project
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer smoke

fmt: install ## Apply Mago formatting
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer fmt

shell: build-8.2 ## Open a shell in the PHP 8.2 image
	$(DOCKER) run --rm -it --security-opt label=disable -u "$$(id -u):$$(id -g)" -e HOME=/tmp \
		-v "$(CURDIR):/app" -w /app rich-editor-auy-php:8.2-bookworm sh

exec: build-8.2 ## Run a command in the PHP 8.2 image (cmd='php -v')
	@test -n "$(cmd)" || { echo "usage: make exec cmd='php -v'"; exit 2; }
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm sh -c "$(cmd)"

clean: ## Remove generated dependencies and caches
	rm -rf vendor build .phpunit.cache
