# Contributing Guide

## 1. Fork the Repository

- Make sure you have a GitHub account.
- Visit the repository's page and click the **Fork** button in the top-right corner.

---

## 2. Clone the Fork

- Clone your forked repository to your local machine:

```bash
git clone https://github.com/YOUR_USERNAME/REPOSITORY_NAME.git
```

---

## 3. Execute the tests

- You need to execute the following command to start running the tests:

```bash
cargo test
```

- If you have successfully completed the tests you are ready to start contributing. 🚀 

---

## 4. Create a New Branch

- Create a new branch according to the guidelines in the following document: [Git Guidelines](GIT_GUIDELINE.md).
- Make sure to base the branch name on the type of change you're making (e.g., `feat/name-related-issue`, `fix/name-related-issue`).

```bash
git checkout -b your-branch-name
```

---

## 5. Make Atomic Commits

- Create atomic commits following the guidelines outlined here: [Git Guidelines](GIT_GUIDELINE.md).
- Each commit should represent a small, focused change. Avoid including multiple unrelated changes in a single commit.

```bash
git add .
git commit -m "type: description"
```

---

## 6. Push Your Changes

- Push the changes to your forked repository:

```bash
git push origin your-branch-name
```

---

## 7. Generate a Pull Request (PR)

- Create a Pull Request (PR) to the **develop** branch of the original repository.
- Follow the PR template below to submit your PR.
- **Important:** If you don’t use the provided PR template properly, your PR will be ignored.

---

## 8. On-Chain Contract Reference

Before making changes to the smart contract logic, please review the [On-Chain Contract Reference](docs/CONTRACT_REFERENCE.md) to understand the public entrypoints, storage model, roles, error codes, and events.
