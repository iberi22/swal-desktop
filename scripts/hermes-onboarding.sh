#!/bin/bash
# ⚡ SWAL Desktop - Hermes Onboarding Helper
# SouthWest AI Labs ⚡

echo "🛰️ Iniciando Onboarding de Hermes Agent..."

# Verificar si hermes está instalado
if ! command -v hermes &> /dev/null; then
    echo "❌ Error: hermes no está en el PATH. ¿Ya ejecutaste el rebuild?"
    exit 1
fi

echo "🔍 Ejecutando 'hermes doctor' para verificar el entorno..."
hermes doctor

echo ""
echo "🚀 Para configurar Hermes con tu proveedor preferido (ej. OpenRouter/DeepSeek):"
echo "Ejecuta: hermes setup"
echo ""
echo "💡 Tip: Usa 'hermes model' para cambiar el modelo por defecto a DeepSeek V4 Flash."
