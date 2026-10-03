package com.intrada.android

import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class ThemeTokenParityTest {
    @Test
    fun everyIosTokenHasAnAndroidCounterpartOrAReason() {
        val missing = iosTokens.filter { it !in doesNotApply && !androidHas(it) }
        assertEquals("Theme.kt lacks these iOS tokens", emptyList<String>(), missing)
    }

    @Test
    fun everyReasonNamesAnIosToken() {
        assertEquals(emptySet<String>(), doesNotApply.keys - iosTokens.toSet())
    }

    @Test
    fun theParserReadsEveryKindOfIosToken() {
        val expected =
            listOf(
                "IntradaColor.paperTop",
                "IntradaColor.marker",
                "LinearGradient.practiceHero",
                "RadialGradient.playerPaper",
                "IntradaFont.Hanken.semibold",
                "IntradaIconSize.hero",
                "IntradaIconSize.maxPoints",
                "IntradaShadow.y",
                "IntradaMotion.fadeUp",
            )
        assertTrue(iosTokens.toString(), iosTokens.containsAll(expected))
    }

    private val doesNotApply =
        mapOf(
            "IntradaIconSize.textStyle" to
                "Android scales by one font scale, not per text style; points times it, capped at maxPoints"
        )

    private val functionOnAndroid =
        mapOf("RadialGradient.playerPaper" to "its radius needs the screen density")

    private val androidGroup =
        mapOf("LinearGradient" to "IntradaGradient", "RadialGradient" to "IntradaGradient")

    private val iosTokens: List<String> by lazy { parsed.map { it.first } }

    private val iosFunctions: Set<String> by lazy {
        parsed.filter { it.second }.map { it.first }.toSet()
    }

    private val parsed: List<Pair<String, Boolean>> by lazy {
        val path = System.getProperty("intrada.iosTheme") ?: error("intrada.iosTheme is not set")
        parseSwiftTokens(File(path).readLines())
    }

    private fun androidHas(token: String): Boolean {
        val parts = token.split(".")
        val group = parts.dropLast(1)
        val root = androidGroup[group.first()] ?: group.first()
        val className = (listOf(root) + group.drop(1)).joinToString("$")
        val type =
            runCatching {
                Class.forName("com.intrada.android.ui.$className", false, javaClass.classLoader)
            }
                .getOrNull() ?: return false
        val name = parts.last()
        val methods = type.declaredMethods.map { it.name.substringBefore('-') }
        return if (token in iosFunctions || token in functionOnAndroid) name in methods
        else
            type.declaredFields.any { it.name == name } ||
                "get${name.replaceFirstChar { it.uppercase() }}" in methods
    }

    private fun parseSwiftTokens(lines: List<String>): List<Pair<String, Boolean>> {
        val open = ArrayDeque<Pair<String, Int>>()
        val tokens = mutableListOf<Pair<String, Boolean>>()
        var depth = 0
        for (line in lines) {
            val code = line.substringBefore("//").trim()
            TYPE.find(code)?.let { open.addLast(it.groupValues[1] to depth + 1) }
            val body = open.lastOrNull()
            if (body != null && body.second == depth) {
                val path = open.joinToString(".") { it.first }
                MEMBER.find(code)?.let {
                    tokens += "$path.${it.groupValues[2]}" to (it.groupValues[1] == "func")
                }
                CASES.find(code)?.let { match ->
                    match.groupValues[1].split(",").forEach {
                        tokens += "$path.${it.trim()}" to false
                    }
                }
            }
            depth += code.count { it == '{' } - code.count { it == '}' }
            while (open.isNotEmpty() && depth < open.last().second) open.removeLast()
        }
        return tokens.distinctBy { it.first }
    }

    private companion object {
        private const val MODIFIERS =
            """(?:(?:@\w+|public|internal|private|fileprivate|final|nonisolated)\s+)*"""
        val TYPE = Regex("""^$MODIFIERS(?:enum|struct|class|extension)\s+(\w+)""")
        val MEMBER = Regex("""^$MODIFIERS(?:static\s+)?(let|var|func)\s+(\w+)""")
        val CASES = Regex("""^case\s+(\w+(?:\s*,\s*\w+)*)$""")
    }
}
