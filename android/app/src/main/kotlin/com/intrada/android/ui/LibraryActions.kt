package com.intrada.android.ui

class LibraryActions(
    val onDismissError: () -> Unit,
    val onAdd: () -> Unit,
    val onOpen: (String) -> Unit,
)
