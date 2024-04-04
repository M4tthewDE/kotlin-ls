class Foo {
    abstract fun onLongClick(view: View)

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
        if (n == 0 || n == 1) {
            return 1
        }

        return n * factorial(n - 1)
    }
}
