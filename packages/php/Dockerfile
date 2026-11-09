# Gate image for the Composer package: validate, audit, PHPUnit and PHPStan.
FROM php:8.4-cli-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends git unzip \
    && rm -rf /var/lib/apt/lists/*

COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

WORKDIR /app
COPY . .

RUN composer install --no-interaction --no-progress

CMD ["sh", "-c", "composer validate --strict && composer audit && composer test && composer analyse"]
