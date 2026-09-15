package com.crux.examples.counter

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel as AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

import com.crux.examples.counter.Event
import com.crux.examples.counter.ViewModel as CruxViewModel

open class Core : AndroidViewModel() {
    private var core: CoreFfi = CoreFfi()

    var view: CruxViewModel by mutableStateOf(
        CruxViewModel.bincodeDeserialize(core.view())
    )
        private set

    init {
        // Фоновый опрос для подхвата входящих изменений из P2P
        viewModelScope.launch {
            while (isActive) {
                delay(300) // каждые 300 мс синхронизируем UI с ядром
                val latest = CruxViewModel.bincodeDeserialize(core.view())
                if (latest != view) {
                    view = latest
                }
            }
        }
    }

    fun update(event: Event) {
        val viewBytes = core.update(event.bincodeSerialize())
        this.view = CruxViewModel.bincodeDeserialize(core.view())
    }
}
