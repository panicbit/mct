module Test_Forge =

(* === VALUE GET TESTS === *)

test [Forge.value_bool] get "true" = {= "true"}
test [Forge.value_bool] get "false" = {= "false"}
test [Forge.value_int] get "42" = {= "42"}
test [Forge.value_int] get "-7" = {= "-7"}
test [Forge.value_double] get "3.141" = {= "3.141"}
test [Forge.value_double] get "-42.7" = {= "-42.7"}
test [Forge.value_string] get "=hallo welt!=" = {= "=hallo welt!="}

test [Forge.var] get "\"my var o' goodness\"" = {"my var o' goodness"}

test [Forge.bool_property] get "B:\"isReallyGood\"=true" = {
  "isReallyGood"
  {"type" = "B"}
  {"value" = "true"}
}

test [Forge.int_property] get "I:\"theAnswer\"=42" = {
  "theAnswer"
  {"type" = "I"}
  {"value" = "42"}
}

test [Forge.double_property] get "D:\"pi\"=3.141" = {
  "pi"
  {"type" = "D"}
  {"value" = "3.141"}
}

test [Forge.property] get "I:\"foo\"=123\n" = {
  "foo"
  {"type" = "I"}
  {"value" = "123"}
}

test [Forge.group_id] get "\"foobar_42-1\"" = {"\"foobar_42-1\""}

let group_example = "\"level_1-1\" {
  # Keksi!
  I:\"keksi\"=777
  \"level_2-1\" {
    # We need to go deeper...
    B:\"godmode\"=true
  }
  # Quuxi
  I:\"quux\"=7
}
"
test Forge.group get group_example =
  {"\"level_1-1\""
    {"#comment" = "Keksi!"}
    {"keksi"
      {"type" = "I"}
      {"value" = "777"}
    }
    {"\"level_2-1\""
      {"#comment" = "We need to go deeper..."}
      {"godmode"
        {"type" = "B"}
        {"value" = "true"}
      }
    }
    {"#comment" = "Quuxi"}
    {"quux"
      {"type" = "I"}
      {"value" = "7"}
    }
  }


let lns_example = "\"default\" {
  # Just a nice number
  I:\"anicenumber\"=42

  # Paradox
  B:\"isABool\"=false

  # isFalse
  B:\"isFalse\"=false

  # OTHER :O
  B:\"another\"=false
}

"

test Forge.lns put lns_example after
  set "/\"default\"/isABool/value" "true"
=  "\"default\" {
  # Just a nice number
  I:\"anicenumber\"=42

  # Paradox
  B:\"isABool\"=true

  # isFalse
  B:\"isFalse\"=false

  # OTHER :O
  B:\"another\"=false
}

"
