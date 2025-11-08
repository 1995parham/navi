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

" Basic cheat elements
syn match cheatTag "^%.*$"
syn match cheatComment "^#.*$"
syn match cheatMetaComment "^;.*$"
syn match cheatExtend "^@.*$"

" Filter lines with special highlighting
syn match cheatFilterKeyword "\(; \(os\|path\|hostname\):\)\@<=.*$" contained
syn match cheatFilter "^; \(os\|path\|hostname\):.*$" contains=cheatFilterKeyword

" Variable references in commands
" <variable> style (implicit dependencies)
syn match cheatVariableRef "<[a-zA-Z0-9_]\+>" containedin=ALL

" $variable style (explicit dependencies) - with high priority
syn match cheatVariableRefDollar "\$[a-zA-Z0-9_]\+" contained

" Variable definition lines: $ variable_name: bash_command --- fzf_options
" Error: variable names containing hyphens
syn match cheatVariableNameError "^\$\s*[a-zA-Z0-9_]*-[a-zA-Z0-9_-]*\s*:"

" Variable marker and name
syn match cheatVariableMarker "^\$" contained nextgroup=cheatVariableName skipwhite
syn match cheatVariableName "\s*[a-zA-Z0-9_]\+\s*" contained nextgroup=cheatVariableColon
syn match cheatVariableColon ":" contained

" The --- delimiter between command and fzf options
syn match cheatVariableDelim "\s---\s" contained

" fzf options after ---
syn region cheatFzfOptions start="---" end="$" contained contains=cheatVariableDelim,cheatFzfOption
syn match cheatFzfOption "\(--[a-zA-Z0-9-]\+\|--multi\|--header-lines\|--delimiter\|--query\|--filter\|--header\|--preview\|--preview-window\|--column\|--map\|--prevent-extra\|--fzf-overrides\|--expand\)" contained

" Bash command part in variable definitions (with both $var and <var> references)
syn region cheatVariableBashCmd start=":\s*" end="\(\s---\|$\)" contained contains=@Shell,cheatVariableRef,cheatVariableRefDollar keepend

" Complete variable definition line
syn region cheatVariableDef start="^\$" end="$" contains=cheatVariableMarker,cheatVariableName,cheatVariableColon,cheatVariableBashCmd,cheatFzfOptions oneline

" Code blocks with bash syntax
syn region cheatCodeBlock start="^```" end="^```" contains=@Shell

" Regular command lines with bash syntax
syn region cheatCommand start="^[^%#;$@`]" end="$" contains=@Shell oneline

" Highlight links
hi def link cheatTag Title
hi def link cheatComment Comment
hi def link cheatMetaComment Comment
hi def link cheatFilter Special
hi def link cheatFilterKeyword String
hi def link cheatExtend Include
hi def link cheatCodeBlock String

" Variable definition parts
hi def link cheatVariableMarker Special
hi def link cheatVariableName Identifier
hi def link cheatVariableColon Delimiter
hi def link cheatVariableNameError Error
hi def link cheatVariableDef Normal

" Variable references
hi def link cheatVariableRef Type
hi def link cheatVariableRefDollar Type

" fzf options
hi def link cheatVariableDelim Operator
hi def link cheatFzfOptions Comment
hi def link cheatFzfOption Special

let &cpo = s:cpo_save
unlet s:cpo_save

let b:current_syntax = "cheat"
