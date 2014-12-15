module Forge =
autoload xfm

let del_str = Util.del_str
let empty = Util.empty_dos
let del_doseol = del /\r?\n/ "\n"

(* === TYPES === *)
let type_bool = store /B/
let type_int = store /I/
let type_double = store /D/
let type_string = store /S/

(* === VALUES === *)

let value_bool = store /true|false/
let value_int = store /[-?0-9]+/
let value_double = store /-?[0-9]+\.[0-9]+/
let value_string = store /[^\r\n]*/

(* TODO: Optional Quotes with whitespace and equals *)
let var = del /\"?/ "\"" . key /[ 'a-zA-Z_-]+/ . del /\"?/ "\""

(* === PROPERTIES === *)

let bool_property = [label "type" . type_bool] . del_str ":" . var . del_str "=" . [label "value" . value_bool]

let int_property = [label "type" . type_int] . del_str ":" . var . del_str "=" . [label "value" . value_int]

let double_property = [label "type" . type_double] . del_str ":" . var . del_str "=" . [label "value" . value_double]

let property = Util.del_opt_ws "  " . (bool_property | int_property | double_property) . del_doseol

(* === BRACES === *)
let del_lbrace = del_str "{" . Util.del_opt_ws ""
let del_rbrace = Util.del_opt_ws "" . del_str "}" . Util.del_opt_ws ""

(* === GROUPS === *)

(* TODO: Allow optional quotes  with containing spaces *)
(*LAST WORKING: let group_id = key /[a-zA-Z0-9_-]+/ *)
(*LAST WORKING2: let group_id = del_str "\"" . key /[ a-zA-Z0-9'_-]+/ . del_str "\"" . Util.del_ws " " *)

let group_id = key /["a-zA-Z0-9_'-]+( ["a-zA-Z0-9_'-]+)*/
let group_start = group_id . del / \{/ " {" . del_doseol
let group_body = [property] | empty| Util.comment
let group_end = del_rbrace . del_doseol

let rec group = [group_start .
              (group_body | (Util.del_opt_ws "" . group) )*
            . group_end]

(* === MAIN === *)

let lns = (group | empty | Util.comment)*

let filter = incl "/config/*.cfg"
           . incl "/config/*/*.cfg"
           . incl "/config/*/*/*.cfg"
let xfm = transform lns filter