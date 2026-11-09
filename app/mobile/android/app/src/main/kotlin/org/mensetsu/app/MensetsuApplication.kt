package org.mensetsu.app

import android.app.Application
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.storage.TokenStorage

class MensetsuApplication : Application() {
    lateinit var tokenStorage: TokenStorage
        private set

    lateinit var networkManager: NetworkManager
        private set

    override fun onCreate() {
        super.onCreate()
        tokenStorage = TokenStorage(this)
        networkManager = NetworkManager(tokenStorage)
    }
}
