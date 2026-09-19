# Applying configuration

After you edit the configuration file, apply the change by doing the following:

- Validate the syntax of the file.
- Reload the service
- Check the log for errors.

Use the reload command to:

- Apply changes without dropping connections.
- Avoid a full restart.

There are three failure modes. First, skipping validation. Second, confusing reload
with restart. Third, leaving a stale entry in the cache.

## 2. Validation

The validator currently reports the line number for each error. The latest release
changes this behavior. Errors include mismatched braces, unknown keys, type errors, etc.
