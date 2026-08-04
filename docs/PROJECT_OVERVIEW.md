# Overlay Forge Project Overview

## Current Shape

Overlay Forge 1.0.1

This release is a private, lightweight launcher. Product features, settings, credentials, databases, generated data, documentation, versions, and releases belong to the Media, Gaming, and Retirement repositories.

The host owns only:

- A closed typed product registry.
- Requested-target dispatch.
- Last-used product resume state.
- A minimal fallback picker.
- Host window state.

It does not dynamically load code, execute commands from SQLite, own a marketplace, update products, or contain product-domain tables.

## Provenance

The product repositories were extracted from `robcross42/overlay-forge` `develop` commit `96aeb79778e148a66b8f88c2b9bcd27b8a415454`. The original application database remains unchanged as an explicit import and rollback source.
