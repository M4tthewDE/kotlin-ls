class Foo {
    lateinit var test: Int

    abstract fun onLongClick(view: View)

    @Bar
    suspend private fun concatenate(str1: String, str2: String): String {
        test + 1
        return str1 + str2
    }
}
