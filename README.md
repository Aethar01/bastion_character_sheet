# Bastion Character Sheet

This is a simple character sheet for the game Bastion.

## Abilities
To add an ability to the ability browser, create a folder "abilities" in the same directory as the executable. The folder should contain files with the .bastion extension and the contents of the file should be formatted as follows:

```
(
  title: "This is an Example Card",
  tags: ("Example1", "Example2"),
  desc: [
    Example description
  ],
  body: [
    *Requires: Example*

    Example body
  ]
),
(
  title: "This is a 2nd Example Card",
  tags: ("Example",),
  desc: [
    Example description
  ],
  body: [
    Example body
  ]
),
```

