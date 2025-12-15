# Implementation Guides

## Overview
Step-by-step guides for troubleshooting and implementing features.

## Authentication & Security

### 📄 [AUTH_401_DIAGNOSTIC_GUIDE.md](./AUTH_401_DIAGNOSTIC_GUIDE.md)
**诊断指南** - How to diagnose 401 authentication errors
- Common causes
- Debug steps
- Network inspection
- Token validation

### 📄 [401_AUTO_LOGOUT_IMPLEMENTATION_COMPLETE.md](./401_AUTO_LOGOUT_IMPLEMENTATION_COMPLETE.md)
**自动登出** - Auto logout on 401 errors
- Implementation details
- API interceptor setup
- State cleanup
- User experience flow

### 📄 [401_SAFETY_VERIFICATION_REPORT.md](./401_SAFETY_VERIFICATION_REPORT.md)
**安全验证** - Safety verification for 401 handling
- Security considerations
- Edge cases tested
- Production readiness checklist
- Best practices

---

## Quick Reference

### When to use each guide:

| Problem | Use This Guide |
|---------|---------------|
| Getting 401 errors | AUTH_401_DIAGNOSTIC_GUIDE.md |
| Need auto-logout feature | 401_AUTO_LOGOUT_IMPLEMENTATION_COMPLETE.md |
| Security audit for 401 handling | 401_SAFETY_VERIFICATION_REPORT.md |

---

## Contributing Guides

When adding a new guide:
1. Use clear, action-oriented title
2. Include problem statement
3. Provide step-by-step solutions
4. Add code examples
5. Include troubleshooting section
6. Update this README
