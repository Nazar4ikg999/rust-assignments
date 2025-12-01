1. Is Rust single-threaded or multi-threaded? Is it synchronous or asynchronous?

Rust підтримує і single-threaded, і multi-threaded програми.
Може працювати і synchronous, і asynchronous, залежно від коду та використаних бібліотек.

⸻

2. What runtime Rust has? Does it use a GC?

Rust не має класичного runtime і не використовує garbage collector.
Управління пам’яттю працює через систему власності (ownership) і borrow checker.

⸻

3. What static typing means? Benefits?

Static typing це коли типи змінних відомі під час компіляції.
Переваги:
	•	менше помилок у runtime
	•	краща оптимізація
	•	швидкість і безпечність

⸻

4. What is immutability? Benefits?

Immutability це незмінність значень за замовчуванням.
Переваги:
	•	безпечніший код
	•	простіше дебагувати
	•	менше неочікуваних побічних ефектів

⸻

5. What are move semantics? Borrowing rules? Benefits?

Move semantics коли значення передаються шляхом “переїзду”, а не копії.
Borrowing тимчасове запозичення значення (mutable або immutable).
Переваги: безпечна робота з пам’яттю без GC.

⸻

6. Traits. How they are used? Difference from interfaces?

Traits це набори поведінки, які тип повинен реалізувати.
Подібні до інтерфейсів, але:
	•	Rust дозволяє реалізовувати trait для зовнішнього типу
	•	є default реалізації функцій

⸻

7. Lifetimes. What problems do they solve?

Lifetimes визначають, як довго існують посилання.
Потрібні, щоб запобігти “висячим” посиланням і проблемам керування пам’яттю.

⸻

8. What are macros? What problems do they solve?

Macros код, який генерує інший код.
Використовуються для:
	•	зменшення дублювання
	•	створення DSL
	•	написання метапрограм

⸻

9. Difference between String and &str (or Vec vs &[u8])? Fat vs thin pointers.
	•	String володіє даними, mutable.
	•	&str позичене посилання, immutable.
Fat pointers містять:
	•	адресу
	•	довжину
Thin pointers лише адресу.

⸻

10. Static and dynamic dispatch.

Static dispatch вибір функції під час компіляції (через generics).
Dynamic dispatch вибір під час виконання (через dyn Trait).
Static швидший, dynamic гнучкіший.