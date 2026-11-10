# syntax=docker/dockerfile:1
ARG PHP_TAG=8.2-cli-alpine
ARG PHP_DIGEST=sha256:212a8b65f28529edcff248b9140edfac3ed8d651917e0eedb1fe6d3fa6950493
FROM php:${PHP_TAG}@${PHP_DIGEST}

ENV COMPOSER_ALLOW_SUPERUSER=1 \
    COMPOSER_HOME=/tmp/composer \
    COMPOSER_ROOT_VERSION=0.1.0 \
    LC_ALL=C \
    TZ=UTC

COPY --from=composer:2@sha256:07cb23c4c9d528524a8b9a67c403d6ad7fef2268651eb1448e3874d40f28b789 /usr/bin/composer /usr/local/bin/composer

RUN apk add --no-cache git unzip autoconf g++ gcc make pkgconf re2c linux-headers \
    && apk del autoconf g++ gcc make pkgconf re2c linux-headers \
    && rm -rf /tmp/pear

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
