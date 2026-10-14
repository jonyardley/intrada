package com.intrada.android.ui

import com.intrada.shared.LibraryItemView

class LibraryActions(
    val onAdd: () -> Unit,
    val onOpen: (LibraryItemView) -> Unit,
)
