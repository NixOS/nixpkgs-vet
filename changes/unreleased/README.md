# Changelogs

To add a changelog, add a Markdown file to a subdirectory depending on the effort required to update to
that version:

- [Major](./major): A large effort. This will cause a version bump from e.g. 0.1.2 to 1.0.0
- [Medium](./medium): Some effort. This will cause a version bump from e.g. 0.1.2 to 1.2.0
- [Minor](./minor): Little/no effort. This will cause a version bump from e.g. 0.1.2 to 0.1.3

Therefore, the versions use [EffVer](https://jacobtomlinson.dev/effver/).

The Markdown file must have the `.md` file ending, and be of the form

```markdown
# Some descriptive title of the change

Optionally more information
```
