if exists("b:current_syntax")
  finish
endif

let b:current_syntax = "m3lc"

syn match lcOperator "\v\:\="
syn match lcOperator "\v\=\>"
syn keyword lcKeyword fn
syn keyword lcTodo TODO
syn match lcComment "\v#.*$" contains=lcTodo

highlight def link lcFunction Function
highlight def link lcKeyword Keyword
highlight def link lcOperator Operator
highlight def link lcComment Comment
highlight def link lcTodo Todo
