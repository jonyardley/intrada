package com.intrada.android.ui

import androidx.annotation.DrawableRes
import com.intrada.android.R

enum class AppTab(val route: String, val label: String, @param:DrawableRes val icon: Int) {
    LIBRARY("library", "Library", R.drawable.ic_tab_library),
    PRACTICE("practice", "Practice", R.drawable.ic_tab_practice),
    ROUTINES("routines", "Routines", R.drawable.ic_tab_routines),
    PROGRESS("progress", "Progress", R.drawable.ic_tab_progress);

    val tag: String
        get() = "tabs.$route"
}
