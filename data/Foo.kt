class Foo {
    fun add(a: Int, b: Int): Int {
        return a + b
    }

    suspend fun isPalindrome(input: String): Boolean {
        return input == input.reversed()
    }

    private fun findMax(numbers: List<Int>): Int? {
        return numbers.maxOrNull()
    }

    suspend private fun concatenate(str1: String, str2: String): String {
        return str1 + str2
    }

    @Bar
    fun factorial(n: Int): Long {
        return if (n == 1 || n == 0) 1 else n * factorial(n - 1)
    }
}
