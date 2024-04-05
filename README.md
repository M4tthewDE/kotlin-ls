# Kotlin language server

# Type inference

https://papl.cs.brown.edu/2014/Type_Inference.html

https://kotlinlang.org/spec/type-inference.html#type-inference

https://www.gecode.org/papers/Tack_PhD_2009.pdf

# Neovim configuration

```lua
local kotlin_ls_config = {
  cmd = { '/path/to/kotlin-ls/executable' },
  cmd_env = { KOTLIN_LS_LOG = '/path/to/server/log' },
  filetypes = { 'kotlin' },
  root_dir = vim.fs.dirname(vim.fs.find({ 'build.gradle.kts' }, { upward = true })[1]),
  on_attach = on_attach,
}

vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = { "*.kt", "*.kts" },
  callback = function()
    vim.lsp.start(kotlin_ls_config)
  end,
})
```
