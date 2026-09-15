package com.crux.examples.counter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp

import com.crux.examples.simplecounter.ui.theme.SimpleCounterTheme
import com.crux.examples.counter.Event
import com.crux.examples.counter.ViewModel as CruxViewModel

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val core = Core()

        setContent {
            SimpleCounterTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    View(
                        view = core.view,
                        onEvent = { event -> core.update(event) }
                    )
                }
            }
        }
    }
}

@Composable
fun View(view: CruxViewModel, onEvent: (Event) -> Unit) {
    var inputText by remember { mutableStateOf("") }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Loro Peer ID: ${view.peerId}", // исправлено: peerId
                    style = MaterialTheme.typography.labelSmall
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = view.content.ifEmpty { "Текст пуст" },
                    style = MaterialTheme.typography.bodyLarge
                )
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        OutlinedTextField(
            value = inputText,
            onValueChange = { inputText = it },
            label = { Text("Введите текст для добавления") },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(16.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            Button(onClick = {
                if (inputText.isNotEmpty()) {
                    onEvent(Event.Append(inputText))
                    inputText = ""
                }
            }) {
                Text("Добавить в CRDT")
            }

            OutlinedButton(onClick = {
                onEvent(Event.Clear) // исправлено: без скобок ()
            }) {
                Text("Очистить")
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    SimpleCounterTheme {
        View(
            view = CruxViewModel(
                content = "Тестовый Loro текст",
                peerId = "12345", // исправлено: peerId
                version = 1UL,
                snapshot = emptyList()
            ),
            onEvent = {}
        )
    }
}
