" Vim syntax file
" Language: Navi cheatsheet
" Maintainer: Parham Alvani
" Latest Revision: 2025

if exists("b:current_syntax")
  finish
endif

let s:cpo_save = &cpo
set cpo&vim

unlet! b:current_syntax
syn include @Shell syntax/sh.vim
unlet! b:current_syntax

syn match cheatTag "^%.*$"
syn match cheatComment "^#.*$"
syn match cheatMetaComment "^;.*$"
syn match cheatVariable "^$.*$"
syn match cheatExtend "^@.*$"

syn match cheatFilterKeyword "\(; \(os\|path\|hostname\):\)\@<=.*$" contained
syn match cheatFilter "^; \(os\|path\|hostname\):.*$" contains=cheatFilterKeyword

syn match cheatVariableDelim "---" contained

syn region cheatVariableDef start="^\$" end="$" contains=cheatVariable,cheatVariableDelim,@Shell oneline

syn region cheatCodeBlock start="^```" end="^```" contains=@Shell

syn region cheatCommand start="^[^%#;$@`]" end="$" contains=@Shell oneline

syn match cheatVariableRef "<[a-zA-Z0-9_]\+>" containedin=ALL

hi def link cheatTag Title
hi def link cheatComment Comment
hi def link cheatMetaComment Comment
hi def link cheatFilter Special
hi def link cheatFilterKeyword String
hi def link cheatVariable Identifier
hi def link cheatVariableDef Identifier
hi def link cheatVariableRef Type
hi def link cheatVariableDelim Operator
hi def link cheatExtend Include
hi def link cheatCodeBlock String

let &cpo = s:cpo_save
unlet s:cpo_save

let b:current_syntax = "cheat"
