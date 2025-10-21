# 💬 Analizador de Feedback con NLP para Seguridad Psicológica

[![Estado del Proyecto: En Desarrollo](https://img.shields.io/badge/Estado-En%20Desarrollo-yellow.svg)](https://github.com/B-GUST/feedback-analyzer)

[cite_start]Este proyecto materializa mi enfoque como Líder de IA[cite: 486]: usar la tecnología no para reemplazar, sino para potenciar la humanidad en el trabajo.

[cite_start]Es una API de servicio que utiliza **Procesamiento de Lenguaje Natural (NLP)** para analizar el feedback de los empleados y detectar patrones que indiquen un riesgo en la **seguridad psicológica** del equipo, un pilar fundamental para la innovación[cite: 360].

---

## 🎯 El Problema de Negocio

La mayoría de los líderes solo obtienen feedback "filtrado" de sus equipos. Las encuestas anónimas son valiosas, pero a menudo son un "cajón de sastre" de texto libre que nadie tiene tiempo de analizar en profundidad.

Como resultado, los problemas de cultura, el miedo a hablar y el inicio del burnout son invisibles hasta que es demasiado tarde. Los líderes carecen de un "sistema de alerta temprana" para la salud cultural de sus equipos.

## 💡 La Solución Técnica

Un microservicio de IA (una API) construido con **FastAPI** que expone un endpoint `/analyze`. Este endpoint recibe un bloque de texto anónimo (el feedback) y devuelve un objeto JSON estructurado con dos análisis clave:

1.  **Análisis de Sentimiento:** (Positivo, Negativo, Neutro) para obtener un pulso general.
2.  **Clasificación de Seguridad Psicológica:** Un modelo entrenado (o que usa *zero-shot classification*) para etiquetar el texto con indicadores de riesgo, como `Miedo a Hablar`, `Cultura de Culpa`, `Falta de Reconocimiento`, etc.

Esto permite a una organización cuantificar lo cualitativo y tomar acciones preventivas.

## ✨ Características Principales

* **API Robusta:** Desarrollada con **FastAPI** para un alto rendimiento y documentación automática (Swagger UI).
* **Análisis de Sentimiento:** Implementación de un modelo pre-entrenado (ej. de Hugging Face) para una clasificación rápida.
* **Clasificación de Riesgo (Seguridad Psicológica):** El núcleo del proyecto, un clasificador de NLP para identificar patrones sutiles en el texto.
* **Contenerización:** Listo para desplegarse como un microservicio usando **Docker**.

## 🛠️ Stack Tecnológico

* **Lenguaje:** `Python`
* **Servidor API:** `FastAPI`, `Uvicorn`
* **Modelado de Datos (API):** `Pydantic`
* **NLP (Modelado):** `Hugging Face Transformers` (para modelos pre-entrenados y *zero-shot*) y/o `spaCy` (para pipelines más ligeros).
* **Despliegue:** `Docker`

## 📈 Estado del Proyecto

* **Fase 1: API y Modelo Baseline (En Progreso)**
    * [ ] Estructuración del proyecto FastAPI.
    * [ ] Definir los modelos Pydantic (Request y Response).
    * [ ] Implementar el endpoint `/analyze`.
    * [ ] Integrar un modelo de análisis de sentimiento (baseline).
* **Fase 2: Modelo de Seguridad Psicológica**
    * [ ] Definir las categorías de "riesgo de seguridad".
    * [ ] [INVESTIGACIÓN] Probar un modelo *Zero-Shot Classification* de Transformers.
    * [ ] (Opcional) Entrenar un clasificador propio si el *zero-shot* no es suficiente.
* **Fase 3: Despliegue**
    * [ ] Escribir el `Dockerfile` de la API.
    * [ ] Crear `docker-compose.yml` para facilitar las pruebas locales.

## 🚀 Cómo Empezar (Próximamente)

```bash
# Instrucciones para clonar y ejecutar el proyecto
git clone [https://github.com/B-GUST/feedback-analyzer.git](https://github.com/B-GUST/feedback-analyzer.git)
cd feedback-analyzer

# (Recomendado) Crear un entorno virtual
python -m venv venv
source venv/bin/activate

pip install -r requirements.txt

# Ejecutar el servidor API
uvicorn main:app --reload
