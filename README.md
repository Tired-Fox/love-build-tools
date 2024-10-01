# Love Build Tools

This a light layer of tooling around lua's [`love`](https://love2d.org/wiki/Main_Page) game engine framework

The idea is to bring together tooling, versioning, lua definitions, building, cross-platform compiling, and much more.

The goal is not to add more on top of `love` but instead make it's workflow easier to manage.

**Potential Features**

- Cross platform build system
    - Include automatic compression of game files and assets and the build of a `fused` executable
- Standardized file structure
    - Breaks from the "everything in one directory" mentality
- Versioning and auto managing the love library install
- Auto lsp setup and support using the `lua2d` addon

## TODO

- [x] Run command
- [ ] Include specific libs
- [ ] Cross platform compilation
- [ ] Custom icons
- [ ] Spinners while building
- [ ] Better error handling

___

## References

- [`love`](https://love2d.org/wiki/Main_Page)
- [`love-api`](https://github.com/love2d-community/love-api)
- [`love2d`](https://github.com/LuaCATS/love2d)
- [`lls-addon-action`](https://github.com/LuaLS/LLS-Addons-Action)
- [`love2d-tl`](https://github.com/MikuAuahDark/love2d-tl)
