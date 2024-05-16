function GetDapConfigs(cwd)
  return {
    ["test"] = {
      program = cwd .. "/target/debug/deps/todo_tui-ff3b97a8cccd3cc1",
      args = {},
    },
  }
end
