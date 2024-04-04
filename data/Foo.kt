package foo.bar;

class Utility {
    fun add(a: Int, b: Int): Int {
        return a + b
    }

    fun isPalindrome(input: String): Boolean {
        return input == input.reversed()
    }

    fun findMax(numbers: List<Int>): Int? {
        return numbers.maxOrNull()
    }

    fun concatenate(str1: String, str2: String): String {
        return str1 + str2
    }

    fun factorial(n: Int): Long {
        return if (n == 1 || n == 0) 1 else n * factorial(n - 1)
    }
}
