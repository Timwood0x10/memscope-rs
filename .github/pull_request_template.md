## 📋 Pull Request Description

**What does this PR do?**
A clear and concise description of what this pull request changes or adds.

**Related issues**
Fixes #(issue number) or Closes #(issue number)

## 🔄 Type of Change

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🔧 Code refactoring (no functional changes)
- [ ] ⚡ Performance improvement
- [ ] 🧪 Test improvement
- [ ] 🏗️ Build/CI changes

## 🧪 Testing

**How has this been tested?**
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed
- [ ] Benchmarks run (if performance-related)

**Test details:**
```bash
# Commands used to test the changes
cargo test
cargo bench
make check
```

**New tests added:**
- [ ] Unit tests for new functionality
- [ ] Integration tests for new features
- [ ] Performance regression tests
- [ ] No tests needed (explain why)

## 📊 Performance Impact

**Does this change affect performance?**
- [ ] No performance impact expected
- [ ] Performance improvement (provide benchmarks)
- [ ] Potential performance regression (explain why acceptable)
- [ ] Performance impact unknown/needs testing

**Benchmark results** (if applicable):
```
Before: ___ms
After:  ___ms
Improvement: ___%
```

## 🔍 Code Quality

**Code review checklist:**
- [ ] Code follows project style guidelines
- [ ] Self-review of code completed
- [ ] Code is well-commented, particularly in hard-to-understand areas
- [ ] No unwrap() calls added without proper expect() messages
- [ ] Error handling is appropriate
- [ ] No clippy warnings introduced

**Documentation:**
- [ ] README updated (if needed)
- [ ] API documentation updated
- [ ] Examples updated (if API changed)
- [ ] Changelog updated

## 🖥️ Compatibility

**Platform testing:**
- [ ] Linux
- [ ] macOS  
- [ ] Windows
- [ ] All platforms (CI will verify)

**Rust version compatibility:**
- [ ] Tested with MSRV (Minimum Supported Rust Version)
- [ ] Tested with stable Rust
- [ ] No new dependencies requiring newer Rust version

## 📋 Additional Information

**Dependencies:**
- [ ] No new dependencies added
- [ ] New dependencies added (list them and justify)
- [ ] Dependencies updated (explain why)

**Breaking changes details** (if applicable):
Describe any breaking changes and migration path for users.

**Future work:**
Any follow-up work that should be done in future PRs.

**Screenshots/Examples** (if applicable):
Add screenshots or code examples showing the changes in action.

---

## ✅ Pre-submission Checklist

- [ ] I have read the [Contributing Guide](CONTRIBUTING.md)
- [ ] My code follows the project's coding standards
- [ ] I have performed a self-review of my own code
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes

---

**Thank you for contributing to memscope-rs! 🙏**

**Note**: This PR will be reviewed as soon as possible. Please be patient and feel free to ping if you don't hear back within a few days.