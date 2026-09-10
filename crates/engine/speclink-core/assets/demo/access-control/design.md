## Context

Shared projects need role-based access control so team leads can restrict who can edit specs vs. who can only view them.

## Goals / Non-Goals

**Goals:**
- Define role hierarchy: viewer, editor, admin
- Implement permission checks on all write operations
- Add role assignment UI in project settings

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Permission Check Middleware

Centralize permission checks in a middleware layer rather than scattering checks across individual commands.

### Role Storage Format

Store roles in the project config file alongside other settings for simplicity and portability.

## Risks / Trade-offs

- Permission cache staleness → Invalidate cache on any role change
- Migration for existing projects → Default all existing users to admin role
