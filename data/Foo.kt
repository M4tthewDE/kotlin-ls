class Foo {
    abstract fun onLongClick(view: View)

    @Bar
    suspend private fun concatenate(str1: String, str2: String): String {
        return str1 + str2
    }
}
