module Mcproperties =
autoload xfm

let del_str = Util.del_str
let rest = /[^\r\n]*/
let doseol = /\r?\n/
let del_doseol = del /\r?\n/

let comment = [label "#comment" . del_str "#" . store rest . del_doseol "\n"]
test comment get "#Test\n" = {"#comment" = "Test"}

let property = [key /[a-z][a-z.-]*/ . del_str "=" . store rest . del_doseol "\n"]
test property get "a-property=a value\n" = {"a-property" = "a value"}

let empty = [Util.del_ws " " . del_doseol "\n"]

let lns = (comment | property | empty)*

let filter = incl "server.properties"
let xfm = transform lns filter
