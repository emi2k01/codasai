# Usage

## Initialize guide

```shell
codasai init [--path]
```

Initializes a codasai project in `--path` or the current directory if `--path` is not passed.

## Preview page

```shell
codasai preview [--no-open] [--no-run-server]
```

Renders the current unsaved page, serves it in a local web server and opens it in the browser.

## Saving a page

```shell
codasai save
```

Saves the current unsaved page and the workspace to git's history.

## Exporting guide

```shell
codasai build [--base-url]
```

Exports the guide under `.codasai/export/`.

Use `--base-url` if you're not serving under your server's root.

Example for hosting under Github Pages:

```shell
codasai build --base-url "/REPOSITORY-NAME"
```

# Building

The following should work

```shell
cargo build
```

# Contributing

## Recomendations

### Front-end

#### Symlinking to this repository's theme

Create a dummy guide with `codasai init dummy` and then

```shell
rm -rf $DUMMY_GUIDE/.codasai/theme/
ln -s $CODASAI/runtime/theme $DUMMY_GUIDE/.codasai/theme
```

The commands above are for creating a symlink to codasai's default theme so
that you can modify the theme under this repository and use the dummy guide to
see your changes.

#### Watching your changes

Install [watchexec](https://github.com/watchexec/watchexec) and then in a dummy guide run

```shell
watchexec -e scss,html,js,md --no-ignore --ignore ".codasai/export" --on-busy-update=restart -- codasai preview --no-open
```

This will rebuild the preview page every time you make a change. You will have
to open yourself the browser and navigate to `http://127.0.0.1:8000/preview`.
