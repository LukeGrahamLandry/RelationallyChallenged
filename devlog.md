
## TODO

- (not, ne, le, ge) in filter predicate 
- whatever the fuck set divide is 
- join ("Inner join required to receive a mark. However, other types will push your mark towards 5")
- cli 
- inline relations
- test invalid input returns error not crash
- operator precedence
- use null as nan, so it propagates in math

## web (Jan 21)



## more sets (Jan 16)

For set difference op, relax takes `∖` or `-` or `except`, so I just map all those to the minus token. 
which is a bit weird cause then you can use all those for scalar subtraction too. 
eh, its not ambiguous so its probably fine. 

for set product, fully evaluate A, and then for each in B, produce |A| variations. 
TODO: what schema to use is interesting. currently just appending B's names&types to A's 
but how are duplicate names dealt with? there's the namespacing thing but what if you product something with itself? 
Need to try out some examples in relax at some point. 

I feel like i need to do namespacing before I can do join. 
i guess the trivial example where you require different field names is fine. 
It's just product and filter, idk why its different. 
i guess it makes it easier to optimise not creating the whole product table, but you'd hope anything real would notice regardless. 

aaaaa very sad day the stupid hourglass thing is a weird unicode character 
so the lexer can't just bump the current index by one each char, needs to see how many utf8 bytes it is. 
which is wierd, i imagined str slicing doing utf8 indices already since they're so excited about it 
but i guess that would be slow since you'd have to iterate up to the starting index every time.
- https://doc.rust-lang.org/book/ch08-02-strings.html#slicing-strings

i can do the dumb product+filter for the easy case where you drop rows that don't have a match in the other table,
but the other case needs special handling anyway. 

## more math (Jan 15)

`+-*/<> or and` are all trivial. I don't care about short-circuiting the and/or cause everything's a pure function. 
I do end up with a lot of repetitive code passing them along my obsessive pipeline which is a bit unfortunate. 
TODO: deal with operator precedence. 

Added more error checking instead of unwraps in compile. 
Math/cmp ops need num args, logic ops need bool args, eq can be any. 
i disallow eq on different types. will never be true so must be a mistake. 
TODO: can you use lt/gt on strings?

## set ops (Jan 13)

Would be really nice to just put things in a hash set but float: !Hash.  
Maybe: `if n.is_nan() { n = f64::NAN; }`
Probably no point to this but it makes me feel better about iffy hash impl for floats
Make any nans the one official bit pattern so they match when cast to u64 and hashed.
For that to make any since need manual eq too since nan!=nan.
Maybe its better to declare Num never nan and have a separate Value::Nan variant
and just always check division (which i dont even have yet anyway)

TODO: move lazy_init comments here. 

For intersection at least i dont have to compute the whole second argument up front, 
can just make the first set and filter the second based on the first. 

TODO: test intersection 

## more parsing and filter (Jan 13)

often need to parse any number of comma seperated things (rows, headers, pi args). 

for parsing projection, you really dont want to deal with knowing the schema of the input table so 
its a good time to move that to the compile step. 

now that im doing select i realize i probably should have called things that evaluate to rows statements instead of expressions.

want to track the type of each column so can check that usage in expressions is valid. 
like you probably shouldn't be allowed to check if a string is larger than a number or whatever.
Its a bit annoying to decide what the input types for table literals are. 
I guess just base it on the types of the first column. 
TODO: i think relax has optional type annotations on table headers. 
TODO: check that all rows in the table match types of the first. 
TODO: do i want implicit nulls if trailing entries are missing from some rows? 

Similar structure for predicates as for relation expressions. Parse to an ast of strings. 
Then compile that into a simpler ast while checking that the types work out. Like the filter 
expression needs to return a bool. Then for exec, expand it into closures that map a row to a Value. 
And then filter is just Iterator::filter using that closure on each row. 
Nice that it doesn't need to allocate a new row. 

TODO: do you need to be able to eval predicate with multiple input rows? 
I guess conceptually join fuses them into one first and then filters that, so it's fine 
but the strat has gotta be to optimise that so you don't actually walk the whole other list for each item.
Maybe that's what indexes are which sounds like too much of a pain since this will presumably only 
be tested on tiny examples. 

## parse it bb (Jan 12)

I want to use the same syntax as relax so its easy to test against but also i dont want to be white space sensitive. 
buts its fine cause when you're done the row header, there's two values next to each-other without a comma. 
just need extra an lookahead. TODO: probably better to represent eof as a token value instead of an error. 

want a function to expect a TokenType and error if not but my choice to have payloads making it hard because 
you can't check equality for an ident/quoted/num, and you can't pass a match expression to a function. 
So options are a: deal with it being ugly, b: make it a macro, c: give Token an optional of all the possible payloads 
(which is just AstValue I guess). for now, ugly but might revisit. 

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
