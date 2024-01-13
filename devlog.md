
## TODO

- parse to ast
- value expressions
- filter
- union
- intersection
- set difference 
- join
- web ui
- cli 
- tests
- error handing
- pretty print ast
- inline relations

## parse it bb (Jan 12)

I want to use the same syntax as relax so its easy to test against but also i dont want to be white space sensitive. 
buts its fine cause when you're done the row header, there's two values next to each-other without a comma. 
just need extra an lookahead. TODO: probably better to represent eof as a token value instead of an error. 

## lex it bb (Jan 10)

I could find some lexer generator thingy, but its just code so lets see how tedious it is. 

At first, I was just re-slicing the string as you go to chop off the first character. 
but you want to remember where the token started, so maybe it's better to track an index. 
annoying that you need to put the lifetime everywhere because it infers lifetime of self 
which isnt quite what i want, its supposed to be tied to the src string. 

Might as well layout some scaffolding for testing. 
I like the idea of programmatically generated relations (like say numbers and their squares)
where I can write normal programs to easily check the query results. 

Trivial lexer sanity test of just making sure my token spans don't lose any characters and don't overlap. 
Successfully caught mistake with numbers dropping the next char because I popped instead of peeked. 

I'm not sure what the syntax is actually supposed to be. 
Relax uses `Name = { fields\n rows }` and assignment example uses `Name (fields) = { rows }` 
which is nice cause the parser doesn't have to care about whitespace. 
So if I want to support the former I need to add a token for linebreaks with is annoying because you 
probably want to ignore it in every other context. 

## Start Vm Structure (Jan 9)

Seems silly to always actually compare the string names of fields. 
Instead, intern them and just have a unique number, only have to look up the real value when displaying to the user. 

You really don't want to produce the whole relation at each step, cause what if you have huge tables. 
Rust's iterators are lazy so use that as the result of a computation. Calling eval doesn't actually do 
anything, it just builds up a big tree of callbacks that the original caller can poll to get the answer to the query. 
So extra memory usage while running a query is O(query depth) instead of O(number of rows). 
It's still doing a bunch of tiny allocations giving each row a new vec at each state in the pipeline. 
TODO: try to do something with arenas since I know I only need a few at a time, and of similar size. 

TODO: realized my grand string interning plan becomes a massive footgun if anyone were to actually use this, 
      Expr is only valid on the specific vm instance that compiled it. 

## What (Jan 9)

- https://dbis-uibk.github.io/relax/landing

I was so ready for this to be boring learn sql class but now they're like task one is write compiler for iterators over tables? I have no idea what any of the words mean but all the operations seem very reasonable? 
Like idk there must be some trick for doing it fast but i feel like shitty interpreter could just do some for loops. 
As long as you iterate lazily, how bad could it be? 

pi, projection: get field from struct / filter fields 
sigma, selection: filter
rho: rename fields of struct
hourglass, join: concat a struct with another that you looked up in a different table? 
                 like zip two tables but then filter out all but one.   
union and intersection: like sets

relation = table = list of tuples 
schema = names for fields of the tuple

I feel like im missing something since they have such fancy words for everything but it 
seems like just tiny functional language. 
