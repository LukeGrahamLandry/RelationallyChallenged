
## TODO

- lex & parse to ast
- value expressions
- filter
- union
- intersection
- join
- web ui
- cli 
- tests
- error handing
- pretty print ast

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

relation = table = list of tupples 
schema = names for fields of the tupple

I feel like im missing something since they have such fancy words for everything but it 
seems like just tiny functional language. 
