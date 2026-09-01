# The Beginning

What on earth is in this project? A load of nonsense, that's what it is. This is my project to implement dexing in Rust. 

### What the dex? 

For those who don't know, Android applications run on a virtual machine. Previously called Dalvik, now called ART (Android Run Time for long). Generally, we write these applications using Kotlin or Java, these are compiled to JVM bytecode, and then dexed. 

Once upon a time, Android ran in incredibly constrained environments. By that I mean old phones with barely any ram and barely a CPU. It was deemed that the JVM was not appropriate for these devices, thus, the Dalvik virtual machine was born. This VM accepted byte code that looks a bit different in comparison to JVM bytecode. It also wasn't stack based, but instead register based*.

* I want to stress that I thought I knew what was going on under the hood on Android. It turns out I was really wrong. I am currently going through a crash course. 

Lets look at the differences. This Kotlin: 

```kotlin
fun add(): Int {
    val a = 3
    val b = 2
    return a + b
}
```

Becomes this JVM bytecode: 

```
public static final int add()
    -- 8 instructions
    0: iconst_3
    1: istore_0
    2: iconst_2
    3: istore_1
    4: iload_0
    5: iload_1
    6: iadd
    7: ireturn
```

Which becomes this Dalvik bytecode: 

```
int KotlinExplorerKt.add()
    -- 4 instructions
    0000: const/4 v0, #int 3 // #3
    0001: const/4 v1, #int 2 // #2
    0002: add-int v2, v0, v1
    0004: return v2
```

Look at that nice compact code. No need to for pushing and popping from the stack. 

Anyway, the origin of this project was: can I implement d8 in Rust? I want to expand my light understanding of Rust and get a bit more into some interesting coding.

I hope to write a lot of this code myself, I do plan to learn along the way. I am sure I will ask AI for some help along the way. At minimum I want to have th architecture nailed down.


Anyway, this concludes some initial ramblings in this dev log.
