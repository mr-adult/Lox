# Lox
This is my implementation of Lox. It is still a work in progress. It is rust-based using a bytecode interpreter.

So far, only expressions and print statements have been implemented. If statements, variables, and more are yet to come.

Lox is a simple scripting language. It uses C-style and enables simple scripts to be run. Lox is dynamically typed. It supports double-precision numbers (double in most programming languages), booleans, and simple string operations like concatenation. Lox is object oriented, so classes can be declared with inheritance hierarchies.

## Syntax
To declare a variable in Lox, we use the 'var' keyword. All statements must end with a semicolon.
```Lox
var i = 0.0;
```

Lox uses 'nil' in place of 'null.'
```Lox
var x = nil;
```

Comments use the /* */ or // syntax, like most C-style languages.
```Lox
/*
This is a multi-line comment.
*/
// This is a single-line comment.
```

A 'print' statement can be used to output to the console:
```Lox
print "Hello world!";
```

### Operators
Lox supports the simple boolean and arithmetic operators. Boolean operators short circuit. Arithmetic operators follow order of operations. Lox does not perform type conversions except for when NOT is performed on a nil value.

Arithmetic (these all report runtime errors if performed on non-numbers):
- plus: '+' - also performs string concatenation
- minus: '-'
- multiply: '*'
- divide: '/'

Boolean (these report runtime errors if performed on non-boolean values):
- and: 'and'
- or: 'or'
- not: '!' - nil is also permitted in not operations. '!nil' is evaluated to true.

```Lox
var num1 = 2.0
var num2 = 42

print (num1 + num2 - 4) * 2 / 4 == 20; // true

print true or false; // true
print false and true; // false
```

Lox also supports the standard comparison operators (only on numbers), which include:
'<', '>', '<=', '>=', '==', and '!=.'

### Variables
Variables in Lox can be reassigned with different types. A variable is not allowed to be declared in the same scope twice and is not allowed to be assigned from its value in an outer scope.

```Lox
var a = 0;
{
    var a = a; // Illegal! Compile error
}

{
    var a = 30; // OK
}

a = false; // OK
a = nil // also OK

var a = 1; // Illegal! Compile error
```

### Control Flow
Lox does not deviate from the standard C control flow statements.

If
```Lox
if (true) {
    print "here";
}
```

While
```Lox
while (false) {
    print "never here";
}
print "here";
```

For
```Lox
for (var i = 0; i < 5; i++) {
    print i;
}
```

### Function
Functions in Lox are declared using the 'fun' keyword.
```Lox
fun fib(n) {
    if (n == 0) { return 0; }
    if (n == 1) { return 1; }
    return fib(n - 1) + fib(n - 2);
}

print fib(4); // 3
```

Lox also supports closures.
```Lox
var a = 0;
fun printTest() {
    print a;
}

printTest(); // 0
```

### Classes
Lox is object-oriented, so it supports classes. They generally follow the traditional rules of inheritance. To create a class, simply call it by name as you would a function. To extend a class, use the '<' symbol. 'init' is the reserved constructor method for a class.

```Lox
class Breakfast {
    init(meat, bread) {
        this.meat = meat;
        this.bread = bread;
    }

    cook() {
        print "Eggs a-fryin'!";
    }

    serve(who) {
        print "Enjoy your " + this.meat + " and " + this.bread + ", " + who + ".";
    }
}

class Brunch < Breakfast { // Brunch extends Breakfast
    init(meat, bread, drink) {
        // we can reference the super class using 'super'
        super.init(meat, bread);
        this.drink = drink;
    }

    drink() {
        print "How about a mimosa?";
    }
}

var benedict = Brunch("ham", "English muffin");
benedict.serve("dear reader"); // Enjoy your ham and English Muffin, dear reader.
```
