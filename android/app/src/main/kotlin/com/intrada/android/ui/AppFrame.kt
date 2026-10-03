package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.intrada.android.core.Store

@Composable
fun AppFrame(store: Store, modifier: Modifier = Modifier) {
    val navController = rememberNavController()
    val entry by navController.currentBackStackEntryAsState()
    val current =
        AppTab.entries.firstOrNull { it.route == entry?.destination?.route } ?: AppTab.LIBRARY
    Column(modifier.fillMaxSize()) {
        NavHost(
            navController,
            startDestination = AppTab.LIBRARY.route,
            modifier = Modifier.weight(1f).consumeWindowInsets(WindowInsets.navigationBars),
        ) {
            composable(AppTab.LIBRARY.route) { LibraryRoute(store) }
            composable(AppTab.PRACTICE.route) { EmptyTab(AppTab.PRACTICE) }
            composable(AppTab.ROUTINES.route) { EmptyTab(AppTab.ROUTINES) }
            composable(AppTab.PROGRESS.route) { EmptyTab(AppTab.PROGRESS) }
        }
        TabBar(current, onSelect = { navController.select(it) })
    }
}

private fun NavHostController.select(tab: AppTab) {
    navigate(tab.route) {
        popUpTo(graph.findStartDestination().id) { saveState = true }
        launchSingleTop = true
        restoreState = true
    }
}

@Composable
private fun EmptyTab(tab: AppTab) {
    ScreenScaffold(tab.label) {}
}

@Composable
private fun TabBar(current: AppTab, onSelect: (AppTab) -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().background(IntradaColor.tabBarFill)) {
        Box(Modifier.fillMaxWidth().height(1.dp).background(IntradaColor.divider))
        Row(
            Modifier.windowInsetsPadding(
                    WindowInsets.navigationBars.union(WindowInsets.displayCutout)
                )
                .selectableGroup(),
            horizontalArrangement = Arrangement.SpaceEvenly,
        ) {
            AppTab.entries.forEach { tab ->
                TabItem(
                    tab,
                    selected = tab == current,
                    onClick = { onSelect(tab) },
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Composable
private fun TabItem(
    tab: AppTab,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val tint = if (selected) IntradaColor.accent else IntradaColor.inkSecondary
    Column(
        modifier
            .heightIn(min = 56.dp)
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onClick,
            )
            .clearAndSetSemantics {
                testTag = tab.tag
                contentDescription = tab.label
                role = Role.Tab
                this.selected = selected
                onClick {
                    onClick()
                    true
                }
            }
            .padding(vertical = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
    ) {
        Image(
            painterResource(tab.icon),
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            colorFilter = ColorFilter.tint(tint),
        )
        BasicText(
            tab.label,
            style = IntradaFont.micro.copy(color = tint, textAlign = TextAlign.Center),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
