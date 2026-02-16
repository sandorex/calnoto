#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <iostream>

// #include "calnoto-core.hh"

int main(int argc, char *argv[]) {
    // std::cout << get_number() << '\n';

    QGuiApplication app(argc, argv);

    QQmlApplicationEngine engine;
    const QUrl url(QStringLiteral("qrc:/main.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);

    engine.load(url);

    return app.exec();
}
