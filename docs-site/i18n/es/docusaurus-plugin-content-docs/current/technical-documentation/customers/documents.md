---
id: documents
title: Gestión de Documentos
sidebar_position: 3
---

# Gestión de Documentos del Cliente

Este documento describe el sistema de gestión de documentos para clientes, incluyendo carga, almacenamiento y recuperación de documentos.

## Tipos de Documentos

### Documentos de Identidad

| Tipo | Descripción | Requerido para KYC |
|------|-------------|--------------------|
| Identificación oficial | DNI, pasaporte, licencia | Sí |
| Selfie | Foto del cliente | Sí |
| Comprobante de domicilio | Recibo de servicios | Según configuración |

### Documentos Corporativos

| Tipo | Descripción | Aplica a |
|------|-------------|----------|
| Escritura constitutiva | Documento de incorporación | Empresas |
| Poder notarial | Representación legal | Empresas |
| Estados financieros | Información financiera | Empresas |

## Arquitectura de Almacenamiento

```
┌─────────────────────────────────────────────────────────────────┐
│                    Panel de Administración                      │
│                    (Carga de documentos)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API GraphQL                                  │
│               (Mutation: uploadDocument)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 document-storage                                │
│           (Servicio de almacenamiento)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Google Cloud Storage / S3                          │
│            (Almacenamiento de archivos)                         │
└─────────────────────────────────────────────────────────────────┘
```

## Operaciones de Documentos

### Subir Documento

1. Navegar al detalle del cliente
2. Seleccionar **Documentos** > **Subir**
3. Seleccionar tipo de documento
4. Arrastrar o seleccionar archivo
5. Confirmar carga

### Formatos Soportados

| Formato | Extensión | Tamaño Máximo |
|---------|-----------|---------------|
| PDF | .pdf | 10 MB |
| Imagen | .jpg, .png | 5 MB |
| Documento | .doc, .docx | 10 MB |

### Via API GraphQL

```graphql
mutation UploadDocument($input: DocumentUploadInput!) {
  documentUpload(input: $input) {
    document {
      id
      filename
      status
      createdAt
    }
  }
}
```

El input incluye:
- `customerId`: ID del cliente
- `documentType`: Tipo de documento
- `file`: Archivo (multipart upload)

## Estados del Documento

| Estado | Descripción |
|--------|-------------|
| PENDING | Cargado, pendiente de revisión |
| APPROVED | Documento validado |
| REJECTED | Documento rechazado |
| EXPIRED | Documento vencido |

## Flujo de Aprobación de Documentos

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Cargado    │───▶│   En Revisión│───▶│  Aprobado/   │
│   (PENDING)  │    │              │    │  Rechazado   │
└──────────────┘    └──────────────┘    └──────────────┘
```

### Aprobar Documento

1. Navegar a la lista de documentos pendientes
2. Revisar documento
3. Hacer clic en **Aprobar** o **Rechazar**
4. Si se rechaza, agregar motivo

### Motivos de Rechazo Comunes

- Documento ilegible
- Documento vencido
- No coincide con información del cliente
- Formato incorrecto

## Consulta de Documentos

### Lista de Documentos del Cliente

```graphql
query GetCustomerDocuments($customerId: ID!) {
  customer(id: $customerId) {
    documents {
      id
      filename
      documentType
      status
      createdAt
      url
    }
  }
}
```

### Descargar Documento

Los documentos se descargan mediante URLs firmadas con tiempo de expiración:

```graphql
query GetDocumentUrl($documentId: ID!) {
  document(id: $documentId) {
    signedUrl(expiresIn: 3600)  # 1 hora
  }
}
```

## Seguridad y Permisos

### Permisos Requeridos

| Operación | Permiso |
|-----------|---------|
| Subir documento | DOCUMENT_CREATE |
| Ver documento | DOCUMENT_READ |
| Aprobar documento | DOCUMENT_UPDATE |
| Eliminar documento | DOCUMENT_DELETE |

### Cifrado

- Documentos cifrados en reposo (encryption at rest)
- Transmisión cifrada (TLS)
- URLs firmadas con expiración

## Retención de Documentos

### Política de Retención

| Tipo | Período de Retención |
|------|---------------------|
| Documentos KYC | 5 años después de cierre de cuenta |
| Documentos de transacción | 7 años |
| Correspondencia | 3 años |

### Archivado

Los documentos antiguos se archivan automáticamente a almacenamiento de bajo costo.

## Integración con KYC

### Documentos de Sumsub

Los documentos subidos durante el proceso de KYC en Sumsub se sincronizan automáticamente:

1. Cliente completa KYC en Sumsub
2. Webhook notifica documentos verificados
3. Sistema descarga y almacena copias
4. Documentos disponibles en el perfil del cliente

### Sincronización Manual

Si es necesario, se puede forzar la sincronización:

1. Navegar al detalle del cliente
2. Sección KYC > **Sincronizar Documentos**

