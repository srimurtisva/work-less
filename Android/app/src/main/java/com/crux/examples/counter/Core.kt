package com.crux.examples.counter

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel as AndroidViewModel

// Импортируем все сгенерированные типы Crux с префиксом com.
import com.crux.examples.counter.Effect
import com.crux.examples.counter.Event
import com.crux.examples.counter.Request
import com.crux.examples.counter.Requests
import com.crux.examples.counter.ViewModel as CruxViewModel

open class Core : AndroidViewModel() {
    private var core: CoreFfi = CoreFfi()

    var view: CruxViewModel by mutableStateOf(
        CruxViewModel.bincodeDeserialize(core.view())
    )
        private set

    fun update(event: Event) {
        val effects = core.update(event.bincodeSerialize())

        val requests = Requests.bincodeDeserialize(effects).value
        for (request in requests) {
            processEffect(request)
        }
    }

    private fun processEffect(request: Request) {
        when (val effect = request.effect) {
            is Effect.Render -> {
                this.view = CruxViewModel.bincodeDeserialize(core.view())
            }
        }
    }
}
