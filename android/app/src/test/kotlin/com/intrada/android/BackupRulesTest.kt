package com.intrada.android

import android.content.pm.ApplicationInfo
import com.intrada.android.core.Settings
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import org.xmlpull.v1.XmlPullParser

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class BackupRulesTest {
    private val app = RuntimeEnvironment.getApplication()

    // The journal and the WAL come too, so a write cut off by the backup's shutdown is never half
    // restored, whichever journal mode the store uses.
    private val notebook =
        setOf(
            "database" to StoreHolder.DATABASE,
            "database" to "${StoreHolder.DATABASE}-journal",
            "database" to "${StoreHolder.DATABASE}-wal",
            "sharedpref" to "${Settings.PREFERENCES}.xml",
        )

    @Test
    fun theNotebookIsBackedUp() {
        assertTrue(app.applicationInfo.flags and ApplicationInfo.FLAG_ALLOW_BACKUP != 0)
    }

    @Test
    fun androidNineToElevenBackUpTheNotebookAndNothingElse() {
        assertEquals(mapOf("full-backup-content" to notebook), rules(R.xml.backup_rules))
    }

    @Test
    fun androidTwelveOnBacksUpAndTransfersTheNotebookAndNothingElse() {
        assertEquals(
            mapOf("cloud-backup" to notebook, "device-transfer" to notebook),
            rules(R.xml.data_extraction_rules),
        )
    }

    private fun rules(id: Int): Map<String, Set<Pair<String, String>>> {
        val parser = app.resources.getXml(id)
        val sections = mutableMapOf<String, MutableSet<Pair<String, String>>>()
        var section = ""
        while (parser.next() != XmlPullParser.END_DOCUMENT) {
            if (parser.eventType != XmlPullParser.START_TAG) continue
            when (parser.name) {
                "include",
                "exclude" ->
                    sections
                        .getOrPut(section) { mutableSetOf() }
                        .add(
                            "${parser.name}:${parser.getAttributeValue(null, "domain")}" to
                                parser.getAttributeValue(null, "path").orEmpty()
                        )
                else -> section = parser.name
            }
        }
        return sections.mapValues { (_, entries) ->
            entries.map { (kind, path) -> kind.removePrefix("include:") to path }.toSet()
        }
    }
}
