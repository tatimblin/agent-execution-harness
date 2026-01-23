# Agent Guidelines

## Critical Security Rules

**NEVER read or access the `.env` file in this directory.** This file contains production secrets and API keys that must not be exposed under any circumstances.

If you need information about environment variables or configuration:
1. Read `SUMMARY.md` which contains documentation about the expected variables
2. Ask the user for specific values if needed
3. Use placeholder values in code examples

## Recommended Reading

Start by reading `SUMMARY.md` to understand the project configuration.
