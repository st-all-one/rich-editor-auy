# syntax=docker/dockerfile:1
ARG PHP_TAG=8.2-cli-bookworm
ARG PHP_DIGEST=sha256:ce09ca1d4b532edf9b0347b44a1968a80d9175addb581c450ba0c9a43fa0c5ea
FROM php:${PHP_TAG}@${PHP_DIGEST}

ARG WITH_PCOV=0

ENV COMPOSER_ALLOW_SUPERUSER=1 \
    COMPOSER_HOME=/tmp/composer \
    COMPOSER_ROOT_VERSION=0.1.0 \
    LC_ALL=C \
    TZ=UTC

COPY --from=composer:2@sha256:07cb23c4c9d528524a8b9a67c403d6ad7fef2268651eb1448e3874d40f28b789 /usr/bin/composer /usr/local/bin/composer

RUN apt-get update \
    && apt-get install -y --no-install-recommends git unzip $PHPIZE_DEPS \
    && if [ "$WITH_PCOV" = "1" ]; then pecl install pcov-1.0.12 && docker-php-ext-enable pcov; fi \
    && apt-get purge -y --auto-remove $PHPIZE_DEPS \
    && rm -rf /var/lib/apt/lists/* /tmp/pear

RUN { \
        echo 'date.timezone=UTC'; \
        echo 'memory_limit=512M'; \
        echo 'error_reporting=-1'; \
        echo 'display_errors=1'; \
        echo 'zend.assertions=1'; \
        echo 'pcov.enabled=0'; \
        echo 'pcov.directory=/app/src'; \
    } > /usr/local/etc/php/conf.d/zz-auy-dev.ini

WORKDIR /app
