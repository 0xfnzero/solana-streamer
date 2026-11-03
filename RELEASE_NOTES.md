# solana-streamer v6.0.0

Released November 3, 2026.

## Changes

Depend on published sol-parser-sdk 0.11.0 and preserve creation transaction versions and ALT keys through the streamer bridge. Remove obsolete parse-status fields and keep third_program provenance.

## Migration

Upgrade direct parser dependencies to 0.11.0. Remove parse_status usage and initialize new creation fields in struct literals. Public event shape changes require this major version.

## Validation

Native streamer and Pump execution replay regressions passed with the registry dependency.
